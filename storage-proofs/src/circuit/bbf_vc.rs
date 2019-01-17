// use std::marker::PhantomData;

// use bellman::{Circuit, ConstraintSystem, SynthesisError};
// use pairing::bls12_381::{Bls12, Fr};
// use pairing::PrimeField;
// use sapling_crypto::circuit::boolean::{self, Boolean};
// use sapling_crypto::circuit::{multipack, num};
// use sapling_crypto::jubjub::JubjubEngine;

// use crate::bbf_vc::BbfVc;
// use crate::circuit::constraint;
// use crate::circuit::kdf::kdf;
// use crate::circuit::sloth;
// use crate::circuit::variables::Root;
// use crate::compound_proof::{CircuitComponent, CompoundProof};
// use crate::drgraph::Graph;
// use crate::fr32::fr_into_bytes;
// use crate::merklepor;
// use crate::parameter_cache::{CacheableParameters, ParameterSetIdentifier};
// use crate::proof::ProofScheme;
// use crate::util::{bytes_into_bits, bytes_into_boolean_vec};

// /// DRG based Proof of Replication.
// ///
// /// # Fields
// ///
// /// * `params` - parameters for the curve
// /// * `sloth_iter` - How many rounds sloth should run for.
// ///
// /// ----> Private `replica_node` - The replica node being proven.
// ///
// /// * `replica_node` - The replica node being proven.
// /// * `replica_node_path` - The path of the replica node being proven.
// /// * `replica_root` - The merkle root of the replica.
// ///
// /// * `replica_parents` - A list of all parents in the replica, with their value.
// /// * `replica_parents_paths` - A list of all parents paths in the replica.
// ///
// /// ----> Private `data_node` - The data node being proven.
// ///
// /// * `data_node_path` - The path of the data node being proven.
// /// * `data_root` - The merkle root of the data.
// /// * `replica_id` - The id of the replica.
// /// * `degree` - The degree of the graph.
// ///
// use crate::circuit::por::{PoRCircuit, PoRCompound};
// use crate::hasher::{Domain, Hasher};

// pub struct BbfVcCircuit<'a, E: JubjubEngine> {
//     params: &'a E::Params,
//     sloth_iter: usize,
//     replica_nodes: Vec<Option<E::Fr>>,
//     replica_nodes_paths: Vec<Vec<Option<(E::Fr, bool)>>>,
//     replica_root: Root<E>,
//     replica_parents: Vec<Vec<Option<E::Fr>>>,
//     replica_parents_paths: Vec<Vec<Vec<Option<(E::Fr, bool)>>>>,
//     data_nodes: Vec<Option<E::Fr>>,
//     data_nodes_paths: Vec<Vec<Option<(E::Fr, bool)>>>,
//     data_root: Root<E>,
//     replica_id: Option<E::Fr>,
//     degree: usize,
//     private: bool,
// }

// impl<'a, E: JubjubEngine> BbfVcCircuit<'a, E> {
//     pub fn synthesize<CS>(
//         mut cs: CS,
//         params: &E::Params,
//         sloth_iter: usize,
//         replica_nodes: Vec<Option<E::Fr>>,
//         replica_nodes_paths: Vec<Vec<Option<(E::Fr, bool)>>>,
//         replica_root: Root<E>,
//         replica_parents: Vec<Vec<Option<E::Fr>>>,
//         replica_parents_paths: Vec<Vec<Vec<Option<(E::Fr, bool)>>>>,
//         data_nodes: Vec<Option<E::Fr>>,
//         data_nodes_paths: Vec<Vec<Option<(E::Fr, bool)>>>,
//         data_root: Root<E>,
//         replica_id: Option<E::Fr>,
//         degree: usize,
//         private: bool,
//     ) -> Result<(), SynthesisError>
//     where
//         E: JubjubEngine,
//         CS: ConstraintSystem<E>,
//     {
//         BbfVcCircuit {
//             params,
//             sloth_iter,
//             replica_nodes,
//             replica_nodes_paths,
//             replica_root,
//             replica_parents,
//             replica_parents_paths,
//             data_nodes,
//             data_nodes_paths,
//             data_root,
//             replica_id,
//             degree,
//             private,
//         }
//         .synthesize(&mut cs)
//     }
// }

// #[derive(Clone)]
// pub struct ComponentPrivateInputs<E: JubjubEngine> {
//     pub comm_r: Option<Root<E>>,
//     pub comm_d: Option<Root<E>>,
// }

// impl<E: JubjubEngine> Default for ComponentPrivateInputs<E> {
//     fn default() -> ComponentPrivateInputs<E> {
//         ComponentPrivateInputs {
//             comm_r: None,
//             comm_d: None,
//         }
//     }
// }

// impl<'a, E: JubjubEngine> CircuitComponent for BbfVcCircuit<'a, E> {
//     type ComponentPrivateInputs = ComponentPrivateInputs<E>;
// }

// pub struct BbfVcCompound<H, G>
// where
//     H: Hasher,
//     G: Graph<H>,
// {
//     // Sad phantom is sad
//     _h: PhantomData<H>,
//     _g: PhantomData<G>,
// }

// impl<E: JubjubEngine, C: Circuit<E>, H: Hasher, G: Graph<H>, P: ParameterSetIdentifier>
//     CacheableParameters<E, C, P> for BbfVcCompound<H, G>
// {
//     fn cache_prefix() -> String {
//         String::from("drg-proof-of-replication")
//     }
// }

// impl<'a, H, G> CompoundProof<'a, Bls12, BbfVc<'a, H, G>, BbfVcCircuit<'a, Bls12>>
//     for BbfVcCompound<H, G>
// where
//     H: 'a + Hasher,
//     G: 'a + Graph<H> + ParameterSetIdentifier + Sync + Send,
// {
//     fn generate_public_inputs(
//         pub_in: &<BbfVc<'a, H, G> as ProofScheme<'a>>::PublicInputs,
//         pub_params: &<BbfVc<'a, H, G> as ProofScheme<'a>>::PublicParams,
//         // We can ignore k because challenges are generated by caller and included
//         // in PublicInputs.
//         _k: Option<usize>,
//     ) -> Vec<Fr> {
//         let replica_id = pub_in.replica_id;
//         let challenges = &pub_in.challenges;
//         let (comm_r, comm_d) = match pub_in.tau {
//             None => (None, None),
//             Some(tau) => (Some(tau.comm_r), Some(tau.comm_d)),
//         };

//         let leaves = pub_params.graph.size();

//         let replica_id_bits = bytes_into_bits(&replica_id.into_bytes());

//         let packed_replica_id =
//             multipack::compute_multipacking::<Bls12>(&replica_id_bits[0..Fr::CAPACITY as usize]);

//         let por_pub_params = merklepor::PublicParams {
//             leaves,
//             private: comm_d.is_none(),
//         };

//         let mut input = Vec::new();
//         input.extend(packed_replica_id.clone());

//         for challenge in challenges {
//             let mut por_nodes = vec![*challenge];
//             let parents = pub_params.graph.parents(*challenge);
//             por_nodes.extend(parents);

//             for node in por_nodes {
//                 let por_pub_inputs = merklepor::PublicInputs {
//                     commitment: comm_r,
//                     challenge: node,
//                 };
//                 let por_inputs = PoRCompound::<H>::generate_public_inputs(
//                     &por_pub_inputs,
//                     &por_pub_params,
//                     None,
//                 );

//                 input.extend(por_inputs);
//             }

//             let por_pub_inputs = merklepor::PublicInputs {
//                 commitment: comm_d,
//                 challenge: *challenge,
//             };

//             let por_inputs =
//                 PoRCompound::<H>::generate_public_inputs(&por_pub_inputs, &por_pub_params, None);
//             input.extend(por_inputs);
//         }
//         input
//     }

//     fn circuit<'b>(
//         public_inputs: &'b <BbfVc<'a, H, G> as ProofScheme<'a>>::PublicInputs,
//         component_private_inputs: <BbfVcCircuit<'a, Bls12> as CircuitComponent>::ComponentPrivateInputs,
//         proof: &'b <BbfVc<'a, H, G> as ProofScheme<'a>>::Proof,
//         public_params: &'b <BbfVc<'a, H, G> as ProofScheme<'a>>::PublicParams,
//         engine_params: &'a <Bls12 as JubjubEngine>::Params,
//     ) -> BbfVcCircuit<'a, Bls12> {
//         let replica_nodes = proof
//             .replica_nodes
//             .iter()
//             .map(|node| Some(node.data.into()))
//             .collect();

//         let replica_nodes_paths = proof
//             .replica_nodes
//             .iter()
//             .map(|node| node.proof.as_options())
//             .collect();

//         let private_data_root = component_private_inputs.comm_d;
//         let private_replica_root = component_private_inputs.comm_r;
//         let replica_root =
//             private_replica_root.unwrap_or_else(|| Root::Val(Some(proof.replica_root.into())));
//         let data_root =
//             private_data_root.unwrap_or_else(|| Root::Val(Some((proof.data_root).into())));
//         let replica_id = Some(public_inputs.replica_id);

//         let replica_parents = proof
//             .replica_parents
//             .iter()
//             .map(|parents| {
//                 parents
//                     .iter()
//                     .map(|(_, parent)| Some(parent.data.into()))
//                     .collect()
//             })
//             .collect();

//         let replica_parents_paths: Vec<Vec<_>> = proof
//             .replica_parents
//             .iter()
//             .map(|parents| {
//                 let p: Vec<_> = parents
//                     .iter()
//                     .map(|(_, parent)| parent.proof.as_options())
//                     .collect();
//                 p
//             })
//             .collect();

//         let data_nodes = proof
//             .nodes
//             .iter()
//             .map(|node| Some(node.data.into()))
//             .collect();

//         let data_nodes_paths = proof
//             .nodes
//             .iter()
//             .map(|node| node.proof.as_options())
//             .collect();

//         BbfVcCircuit {
//             params: engine_params,
//             sloth_iter: public_params.sloth_iter,
//             replica_nodes,
//             replica_nodes_paths,
//             replica_root,
//             replica_parents,
//             replica_parents_paths,
//             data_nodes,
//             data_nodes_paths,
//             data_root,
//             replica_id: replica_id.map(|f| f.into()),
//             degree: public_params.graph.degree(),
//             private: public_inputs.tau.is_none(),
//         }
//     }
// }

// ///
// /// # Public Inputs
// ///
// /// * [0] replica_id/0
// /// * [1] replica_id/1
// /// * [2] replica auth_path_bits
// /// * [3] replica commitment (root hash)
// /// * for i in 0..replica_parents.len()
// ///   * [ ] replica parent auth_path_bits
// ///   * [ ] replica parent commitment (root hash) // Same for all.
// /// * [r + 1] data auth_path_bits
// /// * [r + 2] data commitment (root hash)
// ///
// ///  Total = 6 + (2 * replica_parents.len())
// /// # Private Inputs
// ///
// /// * [ ] replica value/0
// /// * for i in 0..replica_parents.len()
// ///  * [ ] replica parent value/0
// /// * [ ] data value/
// ///
// /// Total = 2 + replica_parents.len()
// ///
// impl<'a, E: JubjubEngine> Circuit<E> for BbfVcCircuit<'a, E> {
//     fn synthesize<CS: ConstraintSystem<E>>(self, cs: &mut CS) -> Result<(), SynthesisError>
//     where
//         E: JubjubEngine,
//     {
//         let params = self.params;

//         let replica_id = self.replica_id;
//         let replica_root = self.replica_root;
//         let data_root = self.data_root;

//         let degree = self.degree;

//         let raw_bytes; // Need let here so borrow in match lives long enough.
//         let replica_id_bytes = match replica_id {
//             Some(replica_id) => {
//                 raw_bytes = fr_into_bytes::<E>(&replica_id);
//                 Some(raw_bytes.as_slice())
//             }
//             // Used in parameter generation or when circuit is created only for
//             // structure and input count.
//             None => None,
//         };

//         // get the replica_id in bits
//         let replica_id_bits =
//             bytes_into_boolean_vec(cs.namespace(|| "replica_id_bits"), replica_id_bytes, 32)?;

//         multipack::pack_into_inputs(
//             cs.namespace(|| "replica_id"),
//             &replica_id_bits[0..Fr::CAPACITY as usize],
//         )?;

//         let replica_root_num = replica_root.allocated(cs.namespace(|| "replica_root"))?;
//         let replica_root_var = Root::Var(replica_root_num);

//         let data_root_num = data_root.allocated(cs.namespace(|| "data_root"))?;
//         let data_root_var = Root::Var(data_root_num);

//         for i in 0..self.data_nodes.len() {
//             let mut cs = cs.namespace(|| format!("challenge_{}", i));
//             // ensure that all inputs are well formed
//             let replica_node_path = &self.replica_nodes_paths[i];
//             let replica_parents_paths = &self.replica_parents_paths[i];
//             let data_node_path = &self.data_nodes_paths[i];

//             let replica_node = &self.replica_nodes[i];
//             let replica_parents = &self.replica_parents[i];
//             let data_node = &self.data_nodes[i];

//             assert_eq!(data_node_path.len(), replica_node_path.len());

//             // Inclusion checks
//             {
//                 let mut cs = cs.namespace(|| "inclusion_checks");

//                 PoRCircuit::synthesize(
//                     cs.namespace(|| "replica_inclusion"),
//                     &params,
//                     *replica_node,
//                     replica_node_path.clone(),
//                     replica_root_var.clone(),
//                     self.private,
//                 )?;

//                 // validate each replica_parents merkle proof
//                 for i in 0..replica_parents.len() {
//                     PoRCircuit::synthesize(
//                         cs.namespace(|| format!("parents_inclusion_{}", i)),
//                         &params,
//                         replica_parents[i],
//                         replica_parents_paths[i].clone(),
//                         replica_root_var.clone(),
//                         self.private,
//                     )?;
//                 }

//                 // validate data node commitment
//                 PoRCircuit::synthesize(
//                     cs.namespace(|| "data_inclusion"),
//                     &params,
//                     *data_node,
//                     data_node_path.clone(),
//                     data_root_var.clone(),
//                     self.private,
//                 )?;
//             }

//             // Encoding checks
//             {
//                 let mut cs = cs.namespace(|| "encoding_checks");
//                 // get the parents into bits
//                 let parents_bits: Vec<Vec<Boolean>> = {
//                     replica_parents
//                         .iter()
//                         .enumerate()
//                         .map(|(i, val)| -> Result<Vec<Boolean>, SynthesisError> {
//                             let mut v = boolean::field_into_boolean_vec_le(
//                                 cs.namespace(|| format!("parents_{}_bits", i)),
//                                 *val,
//                             )?;
//                             // sad padding is sad
//                             while v.len() < 256 {
//                                 v.push(boolean::Boolean::Constant(false));
//                             }
//                             Ok(v)
//                         })
//                         .collect::<Result<Vec<Vec<Boolean>>, SynthesisError>>()?
//                 };

//                 // generate the encryption key
//                 let key = kdf(
//                     cs.namespace(|| "kdf"),
//                     replica_id_bits.clone(),
//                     parents_bits,
//                     degree,
//                 )?;

//                 let decoded = sloth::decode(
//                     cs.namespace(|| "sloth_decode"),
//                     &key,
//                     *replica_node,
//                     self.sloth_iter,
//                 )?;

//                 // TODO this should not be here, instead, this should be the leaf Fr in the data_auth_path
//                 // TODO also note that we need to change/makesurethat the leaves are the data, instead of hashes of the data
//                 let expected = num::AllocatedNum::alloc(cs.namespace(|| "data node"), || {
//                     data_node.ok_or_else(|| SynthesisError::AssignmentMissing)
//                 })?;

//                 // ensure the encrypted data and data_node match
//                 constraint::equal(&mut cs, || "equality", &expected, &decoded);
//             }
//         }
//         // profit!
//         Ok(())
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::bbf_vc;
//     use crate::circuit::test::*;
//     use crate::compound_proof;
//     use crate::drgraph::{graph_height, new_seed, BucketGraph};
//     use crate::fr32::{bytes_into_fr, fr_into_bytes};
//     use crate::hasher::pedersen::*;
//     use crate::porep::PoRep;
//     use crate::proof::ProofScheme;
//     use crate::util::data_at_node;
//     use pairing::Field;
//     use rand::Rand;
//     use rand::{Rng, SeedableRng, XorShiftRng};
//     use sapling_crypto::jubjub::JubjubBls12;

//     #[test]
//     fn bbf_vc_input_circuit_with_bls12_381() {
//         let params = &JubjubBls12::new();
//         let rng = &mut XorShiftRng::from_seed([0x3dbe6259, 0x8d313d76, 0x3237db17, 0xe5bc0654]);

//         let nodes = 12;
//         let degree = 6;
//         let challenge = 2;
//         let sloth_iter = 1;

//         let replica_id: Fr = rng.gen();

//         let mut data: Vec<u8> = (0..nodes)
//             .flat_map(|_| fr_into_bytes::<Bls12>(&Fr::rand(rng)))
//             .collect();

//         // TODO: don't clone everything
//         let original_data = data.clone();
//         let data_node: Option<Fr> = Some(
//             bytes_into_fr::<Bls12>(
//                 data_at_node(&original_data, challenge).expect("failed to read original data"),
//             )
//             .unwrap(),
//         );

//         let sp = bbf_vc::SetupParams {
//             drg: bbf_vc::DrgParams {
//                 nodes,
//                 degree,
//                 expansion_degree: 0,
//                 seed: new_seed(),
//             },
//             sloth_iter,
//         };

//         let pp = bbf_vc::BbfVc::<PedersenHasher, BucketGraph<_>>::setup(&sp)
//             .expect("failed to create bbf_vc setup");
//         let (tau, aux) = bbf_vc::BbfVc::<PedersenHasher, _>::replicate(
//             &pp,
//             &replica_id.into(),
//             data.as_mut_slice(),
//             None,
//         )
//         .expect("failed to replicate");

//         let pub_inputs = bbf_vc::PublicInputs {
//             replica_id: replica_id.into(),
//             challenges: vec![challenge],
//             tau: Some(tau.into()),
//         };
//         let priv_inputs = bbf_vc::PrivateInputs::<PedersenHasher> { aux: &aux };

//         let proof_nc = bbf_vc::BbfVc::<PedersenHasher, _>::prove(&pp, &pub_inputs, &priv_inputs)
//             .expect("failed to prove");

//         assert!(
//             bbf_vc::BbfVc::<PedersenHasher, _>::verify(&pp, &pub_inputs, &proof_nc)
//                 .expect("failed to verify"),
//             "failed to verify (non circuit)"
//         );

//         let replica_node: Option<Fr> = Some(proof_nc.replica_nodes[0].data.into());

//         let replica_node_path = proof_nc.replica_nodes[0].proof.as_options();
//         let replica_root = Root::Val(Some((proof_nc.replica_root).into()));
//         let replica_parents = proof_nc.replica_parents[0]
//             .iter()
//             .map(|(_, parent)| Some(parent.data.into()))
//             .collect();
//         let replica_parents_paths: Vec<_> = proof_nc.replica_parents[0]
//             .iter()
//             .map(|(_, parent)| parent.proof.as_options())
//             .collect();

//         let data_node_path = proof_nc.nodes[0].proof.as_options();
//         let data_root = Root::Val(Some((proof_nc.data_root).into()));
//         let replica_id = Some(replica_id);

//         assert!(
//             proof_nc.nodes[0].proof.validate(challenge),
//             "failed to verify data commitment"
//         );
//         assert!(
//             proof_nc.nodes[0]
//                 .proof
//                 .validate_data(&fr_into_bytes::<Bls12>(&data_node.unwrap())),
//             "failed to verify data commitment with data"
//         );

//         let mut cs = TestConstraintSystem::<Bls12>::new();
//         BbfVcCircuit::synthesize(
//             cs.namespace(|| "bbf_vc"),
//             params,
//             sloth_iter,
//             vec![replica_node],
//             vec![replica_node_path],
//             replica_root,
//             vec![replica_parents],
//             vec![replica_parents_paths],
//             vec![data_node],
//             vec![data_node_path],
//             data_root,
//             replica_id,
//             degree,
//             false,
//         )
//         .expect("failed to synthesize circuit");

//         if !cs.is_satisfied() {
//             println!(
//                 "failed to satisfy: {:?}",
//                 cs.which_is_unsatisfied().unwrap()
//             );
//         }

//         assert!(cs.is_satisfied(), "constraints not satisfied");
//         assert_eq!(cs.num_inputs(), 18, "wrong number of inputs");
//         assert_eq!(cs.num_constraints(), 131216, "wrong number of constraints");

//         assert_eq!(cs.get_input(0, "ONE"), Fr::one());

//         assert_eq!(
//             cs.get_input(1, "bbf_vc/replica_id/input 0"),
//             replica_id.unwrap()
//         );
//     }

//     #[test]
//     fn bbf_vc_input_circuit_num_constraints() {
//         let params = &JubjubBls12::new();
//         let rng = &mut XorShiftRng::from_seed([0x3dbe6259, 0x8d313d76, 0x3237db17, 0xe5bc0654]);

//         // 1 GB
//         let n = (1 << 30) / 32;
//         let m = 6;
//         let tree_depth = graph_height(n);
//         let sloth_iter = 1;

//         let mut cs = TestConstraintSystem::<Bls12>::new();
//         BbfVcCircuit::synthesize(
//             cs.namespace(|| "bbf_vc"),
//             params,
//             sloth_iter,
//             vec![Some(Fr::rand(rng)); 1],
//             vec![vec![Some((Fr::rand(rng), false)); tree_depth]; 1],
//             Root::Val(Some(Fr::rand(rng))),
//             vec![vec![Some(Fr::rand(rng)); m]; 1],
//             vec![vec![vec![Some((Fr::rand(rng), false)); tree_depth]; m]; 1],
//             vec![Some(Fr::rand(rng)); 1],
//             vec![vec![Some((Fr::rand(rng), false)); tree_depth]; 1],
//             Root::Val(Some(Fr::rand(rng))),
//             Some(Fr::rand(rng)),
//             m,
//             false,
//         )
//         .expect("failed to synthesize circuit");

//         assert_eq!(cs.num_inputs(), 18, "wrong number of inputs");
//         assert_eq!(cs.num_constraints(), 363392, "wrong number of constraints");
//     }

//     #[test]
//     #[ignore] // Slow test – run only when compiled for release.
//     fn bbf_vc_test_compound() {
//         let params = &JubjubBls12::new();
//         let rng = &mut XorShiftRng::from_seed([0x3dbe6259, 0x8d313d76, 0x3237db17, 0xe5bc0654]);

//         let nodes = 5;
//         let degree = 2;
//         let challenges = vec![1, 3];
//         let sloth_iter = 1;

//         let replica_id: Fr = rng.gen();
//         let mut data: Vec<u8> = (0..nodes)
//             .flat_map(|_| fr_into_bytes::<Bls12>(&Fr::rand(rng)))
//             .collect();

//         // Only generate seed once. It would be bad if we used different seeds in the same test.
//         let seed = new_seed();

//         let setup_params = compound_proof::SetupParams {
//             vanilla_params: &bbf_vc::SetupParams {
//                 drg: bbf_vc::DrgParams {
//                     nodes,
//                     degree,
//                     expansion_degree: 0,
//                     seed,
//                 },
//                 sloth_iter,
//             },
//             engine_params: params,
//             partitions: None,
//         };

//         let public_params = BbfVcCompound::<PedersenHasher, BucketGraph<_>>::setup(&setup_params)
//             .expect("setup failed");

//         let (tau, aux) = bbf_vc::BbfVc::<PedersenHasher, _>::replicate(
//             &public_params.vanilla_params,
//             &replica_id.into(),
//             data.as_mut_slice(),
//             None,
//         )
//         .expect("failed to replicate");

//         let public_inputs = bbf_vc::PublicInputs::<PedersenDomain> {
//             replica_id: replica_id.into(),
//             challenges,
//             tau: Some(tau),
//         };
//         let private_inputs = bbf_vc::PrivateInputs { aux: &aux };

//         // This duplication is necessary so public_params don't outlive public_inputs and private_inputs.
//         let setup_params = compound_proof::SetupParams {
//             vanilla_params: &bbf_vc::SetupParams {
//                 drg: bbf_vc::DrgParams {
//                     nodes,
//                     degree,
//                     expansion_degree: 0,
//                     seed,
//                 },
//                 sloth_iter,
//             },
//             engine_params: params,
//             partitions: None,
//         };

//         let public_params = BbfVcCompound::<PedersenHasher, BucketGraph<_>>::setup(&setup_params)
//             .expect("setup failed");

//         let proof = BbfVcCompound::<PedersenHasher, _>::prove(
//             &public_params,
//             &public_inputs,
//             &private_inputs,
//             None,
//         )
//         .expect("failed while proving");

//         let (circuit, inputs) = BbfVcCompound::<PedersenHasher, _>::circuit_for_test(
//             &public_params,
//             &public_inputs,
//             &private_inputs,
//         );

//         let mut cs = TestConstraintSystem::new();

//         let _ = circuit.synthesize(&mut cs);
//         assert!(cs.is_satisfied());
//         assert!(cs.verify(&inputs));

//         let verified =
//             BbfVcCompound::<PedersenHasher, _>::verify(&public_params, &public_inputs, &proof)
//                 .expect("failed while verifying");

//         assert!(verified);
//     }
// }
